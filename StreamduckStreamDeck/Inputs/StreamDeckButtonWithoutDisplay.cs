// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using Streamduck.Inputs;

namespace StreamduckStreamDeck.Inputs;

public class StreamDeckButtonWithoutDisplay(int x, int y) : Input(x, y, 1, 1, InputIcon.Button), IInputButton {
	public event Action? ButtonPressed;
	public event Action? ButtonReleased;

	internal void CallPressed() {
		ButtonPressed?.Invoke();
	}

	internal void CallReleased() {
		ButtonReleased?.Invoke();
	}
}