using System;

namespace Streamduck.Attributes; 

/**
 * Make property read only in UI despite having public setter
 */
[AttributeUsage(AttributeTargets.Property)]
public class ReadOnlyAttribute : Attribute { }