pipeline {
    agent {
        label 'hardware-testing'  // Jenkins agent with hardware access
    }
    
    parameters {
        choice(
            name: 'TEST_SUITE',
            choices: ['validation', 'regression', 'performance', 'all'],
            description: 'Test suite to run'
        )
        string(
            name: 'DEVICE_COUNT',
            defaultValue: '2',
            description: 'Number of devices to test'
        )
        booleanParam(
            name: 'FLASH_FIRMWARE',
            defaultValue: false,
            description: 'Flash firmware before testing'
        )
        string(
            name: 'FIRMWARE_PATH',
            defaultValue: '',
            description: 'Path to firmware file (if flashing)'
        )
    }
    
    environment {
        PYTHON_VERSION = '3.9'
        TEST_TIMEOUT = '300'
        MAX_PARALLEL_DEVICES = '4'
        RESULTS_DIR = "test_results_${BUILD_NUMBER}"
    }
    
    options {
        timeout(time: 30, unit: 'MINUTES')
        buildDiscarder(logRotator(numToKeepStr: '50'))
        timestamps()
        ansiColor('xterm')
    }
    
    triggers {
        // Nightly regression tests
        cron('H 2 * * *')
        
        // Poll SCM for changes
        pollSCM('H/15 * * * *')
    }
    
    stages {
        stage('Checkout') {
            steps {
                checkout scm
                
                script {
                    // Set build description
                    currentBuild.description = "Test Suite: ${params.TEST_SUITE}, Devices: ${params.DEVICE_COUNT}"
                }
            }
        }
        
        stage('Setup Environment') {
            steps {
                sh '''
                    # Setup Python virtual environment
                    python3 -m venv venv
                    source venv/bin/activate
                    
                    # Install dependencies
                    pip install --upgrade pip
                    pip install -r test_framework/requirements.txt
                    pip install -r requirements.txt
                '''
                
                // Setup hardware permissions
                sh '''
                    # Ensure proper USB permissions
                    sudo usermod -a -G dialout jenkins || true
                    
                    # Setup udev rules
                    sudo cp jenkins/99-test-devices.rules /etc/udev/rules.d/ || true
                    sudo udevadm control --reload-rules || true
                    sudo udevadm trigger || true
                '''
            }
        }
        
        stage('Hardware Verification') {
            steps {
                script {
                    sh '''
                        source venv/bin/activate
                        
                        # Verify hardware setup
                        python -m test_framework.device_manager --list-devices
                        python validation_scripts/setup_validation.py --hardware-check
                    '''
                }
            }
        }
        
        stage('Build Firmware') {
            when {
                expression { params.FLASH_FIRMWARE == true }
            }
            steps {
                sh '''
                    # Build firmware
                    cargo build --release
                    
                    # Copy to test artifacts
                    mkdir -p test_artifacts
                    cp target/thumbv6m-none-eabi/release/firmware.elf test_artifacts/
                '''
            }
        }
        
        stage('Run Tests') {
            parallel {
                stage('Validation Tests') {
                    when {
                        expression { params.TEST_SUITE in ['validation', 'all'] }
                    }
                    steps {
                        script {
                            def firmwareArg = params.FLASH_FIRMWARE ? "--firmware test_artifacts/firmware.elf" : ""
                            
                            sh """
                                source venv/bin/activate
                                
                                python -m test_framework.ci_integration \\
                                    --config test_framework/ci_configs/validation_config.json \\
                                    --devices ${params.DEVICE_COUNT} \\
                                    --parallel ${env.MAX_PARALLEL_DEVICES} \\
                                    --timeout ${env.TEST_TIMEOUT} \\
                                    --output-dir ${env.RESULTS_DIR}/validation \\
                                    --verbose \\
                                    ${firmwareArg}
                            """
                        }
                    }
                    post {
                        always {
                            // Archive test results
                            archiveArtifacts artifacts: "${env.RESULTS_DIR}/validation/**/*", allowEmptyArchive: true
                            
                            // Publish JUnit results
                            publishTestResults testResultsPattern: "${env.RESULTS_DIR}/validation/*.xml"
                        }
                    }
                }
                
                stage('Regression Tests') {
                    when {
                        expression { params.TEST_SUITE in ['regression', 'all'] }
                    }
                    steps {
                        script {
                            def firmwareArg = params.FLASH_FIRMWARE ? "--firmware test_artifacts/firmware.elf" : ""
                            
                            sh """
                                source venv/bin/activate
                                
                                python -m test_framework.ci_integration \\
                                    --config test_framework/ci_configs/regression_config.json \\
                                    --devices ${params.DEVICE_COUNT} \\
                                    --parallel ${env.MAX_PARALLEL_DEVICES} \\
                                    --timeout ${env.TEST_TIMEOUT} \\
                                    --output-dir ${env.RESULTS_DIR}/regression \\
                                    --verbose \\
                                    ${firmwareArg}
                            """
                        }
                    }
                    post {
                        always {
                            archiveArtifacts artifacts: "${env.RESULTS_DIR}/regression/**/*", allowEmptyArchive: true
                            publishTestResults testResultsPattern: "${env.RESULTS_DIR}/regression/*.xml"
                        }
                    }
                }
                
                stage('Performance Tests') {
                    when {
                        expression { params.TEST_SUITE in ['performance', 'all'] }
                    }
                    steps {
                        script {
                            sh """
                                source venv/bin/activate
                                
                                python -m test_framework.ci_integration \\
                                    --config test_framework/ci_configs/performance_config.json \\
                                    --devices ${params.DEVICE_COUNT} \\
                                    --parallel ${env.MAX_PARALLEL_DEVICES} \\
                                    --timeout 1800 \\
                                    --output-dir ${env.RESULTS_DIR}/performance \\
                                    --verbose
                            """
                        }
                    }
                    post {
                        always {
                            archiveArtifacts artifacts: "${env.RESULTS_DIR}/performance/**/*", allowEmptyArchive: true
                            publishTestResults testResultsPattern: "${env.RESULTS_DIR}/performance/*.xml"
                        }
                    }
                }
            }
        }
        
        stage('Generate Reports') {
            steps {
                script {
                    sh '''
                        source venv/bin/activate
                        
                        # Generate comprehensive report
                        python test_framework/report_consolidator.py \\
                            --input-dir ${RESULTS_DIR} \\
                            --output-dir ${RESULTS_DIR}/consolidated \\
                            --format html,json,junit
                    '''
                }
            }
            post {
                always {
                    // Archive consolidated reports
                    archiveArtifacts artifacts: "${env.RESULTS_DIR}/consolidated/**/*", allowEmptyArchive: true
                    
                    // Publish HTML reports
                    publishHTML([
                        allowMissing: false,
                        alwaysLinkToLastBuild: true,
                        keepAll: true,
                        reportDir: "${env.RESULTS_DIR}/consolidated",
                        reportFiles: '*.html',
                        reportName: 'Hardware Test Report'
                    ])
                }
            }
        }
        
        stage('Performance Analysis') {
            when {
                expression { params.TEST_SUITE in ['performance', 'all'] }
            }
            steps {
                script {
                    sh '''
                        source venv/bin/activate
                        
                        # Analyze performance trends
                        python test_framework/performance_analyzer.py \\
                            --results-dir ${RESULTS_DIR}/performance \\
                            --baseline-dir performance_baselines \\
                            --output ${RESULTS_DIR}/performance_analysis.json \\
                            --threshold-file performance_thresholds.json
                    '''
                }
            }
            post {
                always {
                    archiveArtifacts artifacts: "${env.RESULTS_DIR}/performance_analysis.json", allowEmptyArchive: true
                }
            }
        }
    }
    
    post {
        always {
            script {
                // Cleanup virtual environment
                sh 'rm -rf venv || true'
                
                // Reset hardware devices
                sh '''
                    python3 -m test_framework.device_manager --reset-all || true
                '''
                
                // Clean up old test results
                sh '''
                    find . -name "test_results_*" -type d -mtime +7 -exec rm -rf {} + || true
                    find . -name "*.log" -mtime +7 -delete || true
                '''
            }
        }
        
        success {
            script {
                if (env.BRANCH_NAME == 'main') {
                    // Send success notification for main branch
                    emailext (
                        subject: "✅ Hardware Tests Passed - Build ${BUILD_NUMBER}",
                        body: """
                        Hardware testing pipeline completed successfully!
                        
                        Build: ${BUILD_NUMBER}
                        Branch: ${env.BRANCH_NAME}
                        Test Suite: ${params.TEST_SUITE}
                        Devices: ${params.DEVICE_COUNT}
                        
                        View results: ${BUILD_URL}
                        """,
                        to: "${env.CHANGE_AUTHOR_EMAIL ?: 'team@example.com'}"
                    )
                }
            }
        }
        
        failure {
            script {
                // Send failure notification
                emailext (
                    subject: "❌ Hardware Tests Failed - Build ${BUILD_NUMBER}",
                    body: """
                    Hardware testing pipeline failed!
                    
                    Build: ${BUILD_NUMBER}
                    Branch: ${env.BRANCH_NAME}
                    Test Suite: ${params.TEST_SUITE}
                    Devices: ${params.DEVICE_COUNT}
                    
                    View logs: ${BUILD_URL}console
                    View results: ${BUILD_URL}
                    
                    Please check the test results and logs for details.
                    """,
                    to: "${env.CHANGE_AUTHOR_EMAIL ?: 'team@example.com'}",
                    attachLog: true
                )
                
                // Create JIRA ticket for failures on main branch
                if (env.BRANCH_NAME == 'main') {
                    // This would integrate with JIRA plugin
                    echo "Creating JIRA ticket for main branch failure"
                }
            }
        }
        
        unstable {
            script {
                // Send notification for unstable builds (some tests failed)
                emailext (
                    subject: "⚠️ Hardware Tests Unstable - Build ${BUILD_NUMBER}",
                    body: """
                    Hardware testing pipeline completed with some failures.
                    
                    Build: ${BUILD_NUMBER}
                    Branch: ${env.BRANCH_NAME}
                    Test Suite: ${params.TEST_SUITE}
                    Devices: ${params.DEVICE_COUNT}
                    
                    View results: ${BUILD_URL}
                    """,
                    to: "${env.CHANGE_AUTHOR_EMAIL ?: 'team@example.com'}"
                )
            }
        }
    }
}