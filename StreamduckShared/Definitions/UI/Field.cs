namespace Streamduck.Definitions.UI;

// TODO: Rework this class for use with attributes and reflection based Options
public class Field {
	public string? Title { get; init; }
	public string? Description { get; init; }
	public FieldType Type { get; init; } = new FieldType.Header();
}