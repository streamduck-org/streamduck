using System.Text.Json.Serialization;

namespace Streamduck.Plugins;

public readonly struct NamespacedName {
	[JsonConstructor]
	public NamespacedName(string PluginName, string Name) {
		this.PluginName = PluginName;
		this.Name = Name;
	}

	[JsonInclude] public string PluginName { get; }

	[JsonInclude] public string Name { get; }

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