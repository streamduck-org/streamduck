namespace Streamduck.Cores.Default; 

public class DefaultScreen : Screen {
	public DefaultScreen(ScreenItem?[] items) {
		Items = items;
	}
	
	public override bool CanWrite => true;
	public override ScreenItem?[] Items { get; }
}