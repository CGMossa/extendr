
# It is the intention that this will be generated by the rust #[extendr] in due course.
setClass("Person", representation( pointer = "externalptr" ) )

# # see https://dirk.eddelbuettel.com/code/rcpp/Rcpp-modules.pdf

setMethod("$", "Person", function(x, name) {
    function(...) .Call(paste0("wrap__Person__", name), x@pointer, ...)
} )

setMethod("initialize", "Person", function(.Object, ...) {
    .Object@pointer <- .Call("wrap__Person__new", ...)
    .Object
} )

aux_func <- function() .Call("wrap__aux_func")
