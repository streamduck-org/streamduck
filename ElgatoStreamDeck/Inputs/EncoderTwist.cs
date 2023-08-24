using System;

namespace ElgatoStreamDeck.Inputs;

public sealed class EncoderTwist : Input {
	public override InputType Type => InputType.EncoderTwist;

	public sbyte[] Encoders { get; init; } = Array.Empty<sbyte>();
}

public static class EncoderTwistExtension {
	public static EncoderTwist? AsEncoderTwist(this Input input) => input as EncoderTwist;
}