using System;

namespace Streamduck.Scripting; 

/**
 * Information about a parameter or return from Action or Function
 */
public class DataInfo {
	public Type Type { get; }
	public string Name { get; }
	public string? Description { get; init; }

	public DataInfo(Type type, string name) {
		Type = type;
		Name = name;
	}
}