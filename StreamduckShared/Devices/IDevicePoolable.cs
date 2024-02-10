// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Threading.Tasks;

namespace Streamduck.Devices;

/**
 * Device should implement this interface if it needs to be pooled to receive events from it.
 * You don't have to implement this if your device is event based or it can't generate any events.
 */
public interface IDevicePoolable {
	Task Poll();
}