namespace Streamduck.Definitions.Inputs;

public abstract class Input {
	protected Input(int x, int y, uint w, uint h, InputIcon icon) {
		X = x;
		Y = y;
		W = w;
		H = h;
		Icon = icon;
	}

	public int X { get; }
	public int Y { get; }
	public uint W { get; }
	public uint H { get; }
	public InputIcon Icon { get; }
}