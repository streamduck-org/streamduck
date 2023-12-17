using System.Text.Json.Serialization;
using Streamduck.Plugins;

namespace Streamduck.Devices;

[method: JsonConstructor]
public readonly struct NamespacedDeviceIdentifier(NamespacedName NamespacedName, DeviceIdentifier DeviceIdentifier) {
	[JsonIgnore] public string PluginName => NamespacedName.PluginName;

	[JsonIgnore] public string DriverName => NamespacedName.Name;

	[JsonIgnore] public string Identifier => DeviceIdentifier.Identifier;

	[JsonIgnore] public string Description => DeviceIdentifier.Description;

	public NamespacedName NamespacedName { get; } = NamespacedName;

	public DeviceIdentifier DeviceIdentifier { get; } = DeviceIdentifier;

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