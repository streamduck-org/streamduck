using Avalonia;
using Avalonia.Controls;
using Avalonia.ReactiveUI;
using ReactiveUI;
using Streamduck.UI.ViewModels;
using Streamduck.UI.ViewModels.DeviceList;

namespace Streamduck.UI; 

public partial class MainWindow : ReactiveWindow<MainWindowViewModel> {
	public MainWindow() {
		InitializeComponent();
		this.AttachDevTools();
	}

	protected override void OnClosing(WindowClosingEventArgs e) {
		e.Cancel = true;
		Hide();
	}
}