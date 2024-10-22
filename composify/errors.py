class InvalidTypeAnnotation(TypeError):
    """Raised for invalid type annotation."""

    pass

class MissingReturnTypeAnnotation(InvalidTypeAnnotation):
    """Raised when type annotation for a return value is missing."""

    pass


class MissingParameterTypeAnnotation(InvalidTypeAnnotation):
    """Raised when type annotation for a parameter is missing."""

    pass