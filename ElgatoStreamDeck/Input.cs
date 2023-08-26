namespace ElgatoStreamDeck;

public record Input {
	private Input() { }

	public record ButtonStateChange(bool[] Buttons) : Input;

	public record EncoderStateChange(bool[] Encoders) : Input;

	public record EncoderTwist(sbyte[] Encoders) : Input;

	public record TouchScreenPress(ushort X, ushort Y) : Input;

	public record TouchScreenLongPress(ushort X, ushort Y) : Input;

	public record TouchScreenSwipe(ushort StartX, ushort StartY, ushort EndX, ushort EndY) : Input;
}