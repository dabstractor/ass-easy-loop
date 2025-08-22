#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemError {
    ConfigurationInvalid,
    FlashOperationFailed,
    BootloaderError,
}
