using System;

namespace ElgatoStreamDeck.Inputs;

public sealed class EncoderStateChange : Input {
	public override InputType Type => InputType.EncoderStateChange;

	public bool[] Encoders { get; init; } = Array.Empty<bool>();
}

public static class EncoderStateChangeExtension {
	public static EncoderStateChange? AsEncoderStateChange(this Input input) => input as EncoderStateChange;
}