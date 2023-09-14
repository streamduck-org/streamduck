namespace Streamduck.UI.Models.Devices; 

public class DeviceEntry {
	public string Identifier { get; set; }
	public string Description { get; set; }
	public bool Autoconnect { get; set; }

	public DeviceEntry(string identifier, string description, bool autoconnect) {
		Identifier = identifier;
		Description = description;
		Autoconnect = autoconnect;
	}
}