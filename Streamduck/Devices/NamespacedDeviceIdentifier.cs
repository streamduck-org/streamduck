using System.Text.Json.Serialization;

namespace Streamduck.Devices;

public readonly struct NamespacedDeviceIdentifier {
	[JsonConstructor]
	public NamespacedDeviceIdentifier(NamespacedName NamespacedName, DeviceIdentifier DeviceIdentifier) {
		this.NamespacedName = NamespacedName;
		this.DeviceIdentifier = DeviceIdentifier;
	}

	[JsonIgnore] public string PluginName => NamespacedName.PluginName;

	[JsonIgnore] public string DriverName => NamespacedName.Name;

	[JsonIgnore] public string Identifier => DeviceIdentifier.Identifier;

	[JsonIgnore] public string Description => DeviceIdentifier.Description;

	public NamespacedName NamespacedName { get; }

	public DeviceIdentifier DeviceIdentifier { get; }

	public override string ToString() => $"{DeviceIdentifier} from {NamespacedName}";

	public override int GetHashCode() {
		unchecked {
			var hash = 17;
			hash = hash * 23 + NamespacedName.GetHashCode();
			hash = hash * 23 + DeviceIdentifier.GetHashCode();
			return hash;
		}
	}
}