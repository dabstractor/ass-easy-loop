#[cfg(test)]
mod tests {
    use ass_easy_loop::logging::LogLevel;

    #[test]
    fn test_log_level_copy() {
        let level = LogLevel::Info;
        let level2 = level; // This should work if Copy is implemented
        assert_eq!(level as u8, level2 as u8);
    }
}
