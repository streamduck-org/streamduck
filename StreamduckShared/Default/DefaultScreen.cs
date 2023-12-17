using Streamduck.Cores;

namespace Streamduck.Default; 

public class DefaultScreen(ScreenItem?[] items) : Screen {
	public override bool CanWrite => true;
	public override ScreenItem?[] Items { get; } = items;
}