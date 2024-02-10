// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Collections.Generic;
using System.Linq;
using HidApi;

namespace ElgatoStreamDeck;

/**
 * Instantiates HID library under the hood, only one instance should exist
 */
public class DeviceManager {
	private static readonly ushort ElgatoVendorId = 0x0fd9;

	private static DeviceManager? _instance;

	private readonly SafeHidHandle _hid;

	private DeviceManager() {
		_hid = new SafeHidHandle();
	}

	public IEnumerable<(Kind, string)> ListDevices() => Hid.Enumerate(ElgatoVendorId)
		.Select(info => (info.ProductId.ToKind(), info.SerialNumber));

	public Device ConnectDevice(Kind kind, string serial) {
		if (kind == Kind.Unknown) throw new ArgumentException("Can't connect to unrecognized device kind");

		return new Device(new HidApi.Device(ElgatoVendorId, kind.ToPid(), serial), kind);
	}

	public ConcurrentDevice ConnectDeviceConcurrent(Kind kind, string serial) {
		if (kind == Kind.Unknown) throw new ArgumentException("Can't connect to unrecognized device kind");

		return new ConcurrentDevice(new Device(new HidApi.Device(ElgatoVendorId, kind.ToPid(), serial), kind));
	}

	/**
	 * Should be called when you'll never use the manager again, frees the HID library
	 */
	public void Dispose() {
		_hid.Dispose();
		_instance = null;
	}

	public static DeviceManager Get() {
		_instance ??= new DeviceManager();
		return _instance;
	}
}