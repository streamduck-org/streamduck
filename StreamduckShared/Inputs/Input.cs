namespace Streamduck.Inputs;

public abstract class Input(int x, int y, uint w, uint h, InputIcon icon) {
	public int X { get; } = x;
	public int Y { get; } = y;
	public uint W { get; } = w;
	public uint H { get; } = h;
	public InputIcon Icon { get; } = icon;
}