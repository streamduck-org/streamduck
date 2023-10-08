using System;

namespace Streamduck.Fields.Attributes; 

/**
 * Renames property in UI, if name is empty, title will not be shown
 */
[AttributeUsage(AttributeTargets.Property | AttributeTargets.Field)]
public class NameAttribute : Attribute {
	public NameAttribute(string name) {
		Name = name;
	}
	
	public string Name { get; }
}