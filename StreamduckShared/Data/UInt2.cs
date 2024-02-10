// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

namespace Streamduck.Data;

public struct UInt2(uint x, uint y) {
	public uint X { get; set; } = x;
	public uint Y { get; set; } = y;

	public override string ToString() => $"{{ X: {X}, Y: {Y} }}";

	public override int GetHashCode() {
		unchecked {
			var hash = 17;
			hash = hash * 23 + X.GetHashCode();
			hash = hash * 23 + Y.GetHashCode();
			return hash;
		}
	}
}