// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

namespace Streamduck.Inputs;

public abstract class Input(int x, int y, uint w, uint h, InputIcon icon, bool enabled = true) {
	public int X { get; } = x;
	public int Y { get; } = y;
	public uint W { get; } = w;
	public uint H { get; } = h;
	public InputIcon Icon { get; } = icon;
	public bool Enabled { get; } = enabled;
}