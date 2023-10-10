using System;

namespace Streamduck.Attributes; 

/**
 * Makes property be ignored by UI
 */
[AttributeUsage(AttributeTargets.Property)]
public class IgnoreAttribute : Attribute { }