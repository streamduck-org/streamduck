using System;

namespace Streamduck.Attributes; 

/**
 * Renames property in UI, methods, parameters and return types. If name is empty, title will not be shown
 */
[AttributeUsage(AttributeTargets.Property | AttributeTargets.Field | AttributeTargets.Method 
                | AttributeTargets.Parameter | AttributeTargets.ReturnValue)]
public class NameAttribute : Attribute {
	public NameAttribute(string name) {
		Name = name;
	}
	
	public string Name { get; }
}