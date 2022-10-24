/// Metadata describing the device
pub struct DeviceMetadata {
    /// Unique serial number of the device
    pub serial_number: String,

    /// If the device actually has a screen
    pub has_screen: bool,

    /// Image resolution of buttons
    pub resolution: (u16, u16),

    /// Button layout of the device
    pub layout: ButtonLayout
}

/// Layout of the buttons as an array of button counts in each row
pub struct ButtonLayout(Vec<u8>);