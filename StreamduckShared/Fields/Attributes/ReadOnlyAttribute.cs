using System;

namespace Streamduck.Fields.Attributes; 

/**
 * Make property read only in UI despite having public setter
 */
[AttributeUsage(AttributeTargets.Property)]
public class ReadOnlyAttribute : Attribute { }