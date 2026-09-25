using System;

namespace EticaretApp.Application.Common.Attributes;

[AttributeUsage(AttributeTargets.Class, Inherited = false, AllowMultiple = false)]
public sealed class TransactionalAttribute : Attribute { }
