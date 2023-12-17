using System;

namespace Streamduck.Attributes;

/**
 * Include non-public property in UI
 */
[AttributeUsage(AttributeTargets.Property)]
public class IncludeAttribute(bool write = false) : Attribute {
	public bool WriteAllowed { get; } = write;
}