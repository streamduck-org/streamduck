// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Text.Json.Serialization;

namespace Streamduck.Devices;

[method: JsonConstructor]
public readonly struct DeviceIdentifier(string Identifier, string Description) {
	public string Identifier { get; } = Identifier;

	public string Description { get; } = Description;

	public override string ToString() => $"{Identifier} ({Description})";

	public override int GetHashCode() {
		unchecked {
			var hash = 17;
			hash = hash * 23 + Identifier.GetHashCode();
			hash = hash * 23 + Description.GetHashCode();
			return hash;
		}
	}
}