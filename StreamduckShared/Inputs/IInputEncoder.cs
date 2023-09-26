using System;

namespace Streamduck.Inputs;

public interface IInputEncoder {
	event Action<int>? EncoderTwisted;
}