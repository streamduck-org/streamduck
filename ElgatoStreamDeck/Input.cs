// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

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