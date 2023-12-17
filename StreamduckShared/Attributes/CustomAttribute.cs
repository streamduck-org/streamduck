using System;

namespace Streamduck.Attributes; 

/**
 * Will display field using the custom Avalonia Control type,
 * constructor with type of the property is expected as the only parameter.
 * Property title and description if provided will be shown above the control.
 * <remarks>
 * Use [Name("")] to remove title.
 * </remarks>
 */
[AttributeUsage(AttributeTargets.Property)]
public class CustomAttribute(Type userControlType) : Attribute {
	public Type UserControlType { get; } = userControlType;
}