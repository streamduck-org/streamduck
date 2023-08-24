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