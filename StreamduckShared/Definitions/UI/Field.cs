namespace StreamduckPlugin.Definitions.UI; 

public class Field {
	public string[]? ValuePath { get; init; }
	public string? Title { get; init; }
    public string? Description { get; init; }
    public FieldType Type { get; init; } = new FieldType.Header();
}