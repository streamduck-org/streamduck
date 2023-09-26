using System.Text.Json.Serialization;

namespace Streamduck.Devices;

public readonly struct DeviceIdentifier {
	[JsonConstructor]
	public DeviceIdentifier(string Identifier, string Description) {
		this.Identifier = Identifier;
		this.Description = Description;
	}

	public string Identifier { get; }

	public string Description { get; }

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