// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using HidApi;
using Microsoft.Win32.SafeHandles;

namespace ElgatoStreamDeck;

public class SafeHidHandle : SafeHandleZeroOrMinusOneIsInvalid {
	public SafeHidHandle() : base(true) {
		SetHandle(1);
		Hid.Init();
	}

	public override bool IsInvalid => false;

	protected override bool ReleaseHandle() {
		Hid.Exit();

		return true;
	}
}