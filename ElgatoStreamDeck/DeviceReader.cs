// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Collections.Generic;

namespace ElgatoStreamDeck;

using DeviceInput = Input;

public class DeviceReader {
	private readonly bool[] _buttonStates;
	private readonly IDevice _device;
	private readonly bool[] _encoderStates;

	public DeviceReader(IDevice device) {
		_device = device;
		_buttonStates = new bool[_device.Kind().KeyCount()];
		_encoderStates = new bool[_device.Kind().EncoderCount()];
	}

	/**
	 * Blocking, should be ran in another thread
	 */
	public IEnumerable<Input> Read() {
		if (_device.ReadInput() is not { } input) yield break;

		switch (input) {
			case DeviceInput.ButtonStateChange buttonStateChange: {
				for (var i = 0; i < buttonStateChange.Buttons.Length; i++) {
					var oldState = _buttonStates[i];
					var newState = buttonStateChange.Buttons[i];

					if (oldState == newState) continue;

					_buttonStates[i] = newState;
					yield return newState
						? new Input.ButtonPressed((ushort)i)
						: new Input.ButtonReleased((ushort)i);
				}

				break;
			}

			case DeviceInput.EncoderStateChange encoderStateChange: {
				for (var i = 0; i < encoderStateChange.Encoders.Length; i++) {
					var oldState = _encoderStates[i];
					var newState = encoderStateChange.Encoders[i];

					if (oldState == newState) continue;

					_encoderStates[i] = newState;

					yield return newState
						? new Input.EncoderPressed((ushort)i)
						: new Input.EncoderReleased((ushort)i);
				}

				break;
			}

			case DeviceInput.EncoderTwist encoderTwist: {
				for (var i = 0; i < encoderTwist.Encoders.Length; i++)
					if (encoderTwist.Encoders[i] != 0)
						yield return new Input.EncoderTwist((ushort)i, encoderTwist.Encoders[i]);

				break;
			}

			case DeviceInput.TouchScreenPress touchScreenPress: {
				yield return new Input.TouchScreenPress(touchScreenPress.X, touchScreenPress.Y);

				break;
			}

			case DeviceInput.TouchScreenLongPress touchScreenLongPress: {
				yield return new Input.TouchScreenLongPress(touchScreenLongPress.X, touchScreenLongPress.Y);

				break;
			}

			case DeviceInput.TouchScreenSwipe touchScreenSwipe: {
				yield return new Input.TouchScreenSwipe(
					touchScreenSwipe.StartX, touchScreenSwipe.StartY,
					touchScreenSwipe.EndX, touchScreenSwipe.EndY
				);

				break;
			}
		}
	}

	public record Input {
		private Input() { }

		public record ButtonPressed(ushort key) : Input;

		public record ButtonReleased(ushort key) : Input;

		public record EncoderPressed(ushort encoder) : Input;

		public record EncoderReleased(ushort encoder) : Input;

		public record EncoderTwist(ushort encoder, sbyte value) : Input;

		public record TouchScreenPress(ushort X, ushort Y) : Input;

		public record TouchScreenLongPress(ushort X, ushort Y) : Input;

		public record TouchScreenSwipe(ushort StartX, ushort StartY, ushort EndX, ushort EndY) : Input;
	}
}