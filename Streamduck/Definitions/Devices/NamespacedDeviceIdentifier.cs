namespace Streamduck.Definitions.Devices;

public readonly struct NamespacedDeviceIdentifier {
	public NamespacedDeviceIdentifier(NamespacedName driver, DeviceIdentifier identifier) {
		NamespacedName = driver;
		DeviceIdentifier = identifier;
	}

	public string PluginName => NamespacedName.PluginName;

	public string DriverName => NamespacedName.Name;

	public string Identifier => DeviceIdentifier.Identifier;

	public string Description => DeviceIdentifier.Description;
	
	public NamespacedName NamespacedName { get; }
	
	public DeviceIdentifier DeviceIdentifier { get; }

	public override string ToString() => $"ID '{DeviceIdentifier.Identifier}' ({DeviceIdentifier.Description}) from " +
	                                     $"{NamespacedName.Name} ({NamespacedName.PluginName})";
}