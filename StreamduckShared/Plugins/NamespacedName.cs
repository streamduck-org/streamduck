// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

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