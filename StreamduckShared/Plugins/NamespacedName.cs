using System.Text.Json.Serialization;

namespace Streamduck.Plugins;

[method: JsonConstructor]
public readonly struct NamespacedName(string PluginName, string Name) {
	[JsonInclude] public string PluginName { get; } = PluginName;

	[JsonInclude] public string Name { get; } = Name;

	public override string ToString() => $"{Name} ({PluginName})";

	public override int GetHashCode() {
		unchecked {
			var hash = 17;
			hash = hash * 23 + PluginName.GetHashCode();
			hash = hash * 23 + Name.GetHashCode();
			return hash;
		}
	}
}