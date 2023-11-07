namespace Streamduck.Cores; 

/**
 * Screen that can contain screen items
 */
public abstract class Screen {
	public abstract bool CanWrite { get; }
	public abstract ScreenItem?[] Items { get; }
	
}