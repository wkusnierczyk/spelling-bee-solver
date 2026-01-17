import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.window.CanvasBasedWindow

@OptIn(ExperimentalComposeUiApi::class)
fun main() {
    // "title" sets the browser tab title
    // removing "canvasElementId" lets Compose create and attach the canvas automatically
    CanvasBasedWindow(title = "Spelling Bee Solver") {
        App()
    }
}
