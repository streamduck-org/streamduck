using System;

namespace Streamduck.Definitions.Inputs;

public interface IInputEncoder {
	event Action<int>? EncoderTwisted;
}