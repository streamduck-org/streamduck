using Streamduck.Attributes;

namespace StreamduckStreamDeck;

public class StreamDeckDeviceOptions {
	[Header("Screen Controls")]
	[Description("Adjusts screen brightness of the device")]
	public int ScreenBrightness { get; set; } = 100;
}