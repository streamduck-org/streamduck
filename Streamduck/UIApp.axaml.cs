using System;
using System.Threading;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Markup.Xaml;

namespace Streamduck; 

public partial class UIApp : Application {
	public App? StreamduckApp { get; set; }
	public Window? MainWindow { get; set; }

	public CancellationTokenSource? CancellationTokenSource;
	
	public override void Initialize() {
		AvaloniaXamlLoader.Load(this);
	}

	public void OpenUI(object? sender, EventArgs eventArgs) {
		if (MainWindow!.IsVisible) MainWindow!.Hide();
		else MainWindow!.Show();
	}
	
	public void Exit(object? sender, EventArgs eventArgs) {
		CancellationTokenSource?.Cancel();
	}
}