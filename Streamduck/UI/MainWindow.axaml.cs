using Avalonia;
using Avalonia.Controls;
using Avalonia.Markup.Xaml;

namespace Streamduck.UI; 

public partial class MainWindow : Window {
	public MainWindow() {
		InitializeComponent();
	}

	protected override void OnClosing(WindowClosingEventArgs e) {
		e.Cancel = true;
		Hide();
	}
}