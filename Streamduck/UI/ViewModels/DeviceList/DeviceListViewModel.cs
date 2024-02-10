// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using ReactiveUI;
using Streamduck.Devices;

namespace Streamduck.UI.ViewModels.DeviceList;

public class DeviceListViewModel : ViewModelBase, IRoutableViewModel {
	public DeviceListViewModel(IScreen hostScreen) {
		HostScreen = hostScreen;
		Devices.CollectionChanged += (_, _) => this.RaisePropertyChanged(nameof(IsEmpty));

		if (Application.Current is not UIApp app) return;
		if (app.StreamduckApp is not { } streamduck) return;

		streamduck.DeviceAppeared += device => {
			lock (Devices) {
				if (DevicesContains(device)) return;
				Devices.Add(new DeviceEntryViewModel(device, false, HostScreen));
			}
		};

		streamduck.DeviceDisappeared += device => {
			lock (Devices) {
				if (!DevicesContains(device)) return;
				RemoveDevice(device);
			}
		};

		streamduck.DeviceConnected += (device, _) => {
			lock (Devices) {
				if (DevicesContains(device))
					foreach (var entry in DevicesList(device)) {
						entry.Connected = true;
					}
				else
					Devices.Add(new DeviceEntryViewModel(device, true, HostScreen));
			}
		};

		streamduck.DeviceDisconnected += device => {
			lock (Devices) {
				RemoveDevice(device);
			}
		};

		streamduck.RefreshDevices().Wait();
	}

	public ObservableCollection<DeviceEntryViewModel> Devices { get; set; } = [];
	public bool IsEmpty => Devices.Count <= 0;
	public string UrlPathSegment => "devices";
	public IScreen HostScreen { get; }

	private bool DevicesContains(NamespacedDeviceIdentifier deviceIdentifier) =>
		Devices.Any(entry => entry.OriginalIdentifier.Equals(deviceIdentifier));

	private IEnumerable<DeviceEntryViewModel> DevicesList(NamespacedDeviceIdentifier deviceIdentifier) =>
		Devices.Where(entry => entry.OriginalIdentifier.Equals(deviceIdentifier));

	private void RemoveDevice(NamespacedDeviceIdentifier deviceIdentifier) {
		foreach (var entry in DevicesList(deviceIdentifier).ToArray()) {
			Devices.Remove(entry);
		}
	}

	public async Task RefreshDevices() {
		if (Application.Current is not UIApp app) return;
		if (app.StreamduckApp is not { } streamduck) return;
		await streamduck.RefreshDevices();
	}
}