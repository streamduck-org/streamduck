// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using Streamduck.Devices;
using Streamduck.Inputs;

namespace Streamduck.UI.Design;

public class ShellDevice(DeviceIdentifier identifier) : Device(identifier) {
	public override Input[] Inputs { get; } = [
		new ShellInput(0, 0, 10, 10, InputIcon.Button),
		new ShellInput(5, 10, 10, 10, InputIcon.Button),
		new ShellInput(10, 0, 10, 10, InputIcon.Button),
		new ShellInput(8, 20, 10, 10, InputIcon.Button),
		new ShellInput(0, -5, 10, 5, InputIcon.Button),
		new ShellInput(10, -5, 10, 5, InputIcon.Button),
		new ShellInput(20, -5, 10, 5, InputIcon.Button)
	];
}