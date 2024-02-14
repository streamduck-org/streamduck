// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Reactive;
using System.Threading.Tasks;
using Avalonia;
using ReactiveUI;
using Streamduck.Configuration;
using Streamduck.Devices;
using Streamduck.UI.ViewModels.DeviceEditor;

namespace Streamduck.UI.ViewModels.DeviceList;

public class DeviceEntryViewModel : ViewModelBase {
	private readonly IScreen _hostScreen;

	private bool _autoconnect;

	private bool _connected;

	public DeviceEntryViewModel(NamespacedDeviceIdentifier originalIdentifier, bool connected, IScreen hostScreen) {
		OriginalIdentifier = originalIdentifier;
		_autoconnect = Config.IgnorantGet()?.AutoconnectDevices.Contains(originalIdentifier) ?? false;
		_connected = connected;
		_hostScreen = hostScreen;

		OpenDevice = ReactiveCommand.CreateFromTask(
			async () => {
				if (Application.Current is not UIApp app) return;
				if (app.StreamduckApp is not { } streamduck) return;

				if (!streamduck.ConnectedDevices.ContainsKey(originalIdentifier))
					await streamduck.ConnectDevice(originalIdentifier);

				streamduck.ConnectedDevices.TryGetValue(originalIdentifier, out var deviceCore);

				_hostScreen.Router.Navigate.Execute(
					new DeviceEditorViewModel(_hostScreen, deviceCore));
			});
	}

	public ReactiveCommand<Unit, Unit> OpenDevice { get; }

	public NamespacedDeviceIdentifier? OriginalIdentifier { get; }

	public string? Identifier => OriginalIdentifier?.Identifier;
	public string? Description => OriginalIdentifier?.Description;

	public bool AutoConnect {
		get => _autoconnect;
		set {
			this.RaiseAndSetIfChanged(ref _autoconnect, value);

			if (Application.Current is not UIApp app) return;
			if (app.StreamduckApp is not { } streamduck) return;

			_ = Task.Run(async () => {
				if (value)
					await (Config.IgnorantGet()?.AddDeviceToAutoconnect(OriginalIdentifier!.Value) ??
					       Task.CompletedTask);
				else
					await (Config.IgnorantGet()?.RemoveDeviceFromAutoconnect(OriginalIdentifier!.Value) ??
					       Task.CompletedTask);

				await streamduck.RefreshDevices();
			});
		}
	}

	public bool Connected {
		get => _connected;
		set => this.RaiseAndSetIfChanged(ref _connected, value);
	}

	public override bool Equals(object? obj) => obj is DeviceEntryViewModel drhs &&
	                                            drhs.OriginalIdentifier.Equals(OriginalIdentifier);

	public override int GetHashCode() => OriginalIdentifier.GetHashCode();
}