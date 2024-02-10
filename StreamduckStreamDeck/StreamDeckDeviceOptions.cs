// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using Streamduck.Attributes;

namespace StreamduckStreamDeck;

public class StreamDeckDeviceOptions {
	[Header("Screen Controls")]
	[Description("Adjusts screen brightness of the device")]
	public int ScreenBrightness { get; set; } = 100;
}