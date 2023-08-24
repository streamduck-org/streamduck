namespace Streamduck.Definitions.Devices;

public readonly struct NamespacedDeviceIdentifier {
	private readonly NamespacedName _driver;
	private readonly DeviceIdentifier _identifier;

	public NamespacedDeviceIdentifier(NamespacedName driver, DeviceIdentifier identifier) {
		_driver = driver;
		_identifier = identifier;
	}

	public string PluginName => _driver.PluginName;

	public string DriverName => _driver.Name;

	public string Identifier => _identifier.Identifier;

	public string Description => _identifier.Description;
}