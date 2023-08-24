namespace Streamduck.Definitions.Inputs;

public struct Input {
	public int X { get; init; }
	public int Y { get; init; }
	public uint W { get; init; }
	public uint H { get; init; }
	public InputIcon Icon { get; init; }
	public InputBehavior[] Behaviors { get; init; }
	public (uint, uint)? Resolution { get; init; }
}