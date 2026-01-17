import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import io.ktor.client.*
import io.ktor.client.call.*
import io.ktor.client.plugins.contentnegotiation.*
import io.ktor.client.request.*
import io.ktor.client.statement.*
import io.ktor.serialization.kotlinx.json.*
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.coroutines.launch

@Serializable
data class SolveRequest(
    val letters: String,
    val present: String
)

@Composable
fun App() {
    MaterialTheme {
        Surface(modifier = Modifier.fillMaxSize(), color = MaterialTheme.colors.background) {
            SolverScreen()
        }
    }
}

@Composable
fun SolverScreen() {
    var letters by remember { mutableStateOf("") }
    var present by remember { mutableStateOf("") }
    var results by remember { mutableStateOf<List<String>>(emptyList()) }
    var errorMessage by remember { mutableStateOf<String?>(null) }
    var isLoading by remember { mutableStateOf(false) }

    val scope = rememberCoroutineScope()
    
    // Create Client
    val client = remember {
        HttpClient {
            install(ContentNegotiation) {
                json(Json { ignoreUnknownKeys = true })
            }
        }
    }

    Column(
        modifier = Modifier.padding(16.dp).fillMaxWidth(),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        Text("Spelling Bee Solver", style = MaterialTheme.typography.h4)
        
        OutlinedTextField(
            value = letters,
            onValueChange = { letters = it },
            label = { Text("Available Letters (e.g. 'abcdefg')") },
            modifier = Modifier.fillMaxWidth()
        )
        
        OutlinedTextField(
            value = present,
            onValueChange = { present = it },
            label = { Text("Obligatory Letter (e.g. 'a')") },
            modifier = Modifier.fillMaxWidth()
        )
        
        Button(
            onClick = {
                scope.launch {
                    isLoading = true
                    errorMessage = null
                    try {
                        val response = client.post("http://localhost:8080/solve") {
                            header("Content-Type", "application/json")
                            setBody(SolveRequest(letters, present))
                        }
                        
                        if (response.status.value in 200..299) {
                            results = response.body()
                        } else {
                            errorMessage = "Error: ${response.bodyAsText()}"
                        }
                    } catch (e: Exception) {
                        errorMessage = "Failed to connect: ${e.message}"
                    } finally {
                        isLoading = false
                    }
                }
            },
            enabled = !isLoading && letters.isNotEmpty() && present.isNotEmpty()
        ) {
            if (isLoading) {
                CircularProgressIndicator(modifier = Modifier.size(20.dp), color = Color.White)
            } else {
                Text("Solve")
            }
        }
        
        if (errorMessage != null) {
            Text(errorMessage!!, color = Color.Red)
        }
        
        Divider()
        
        Text("Found Words: ${results.size}", style = MaterialTheme.typography.subtitle1)
        
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(bottom = 16.dp)
        ) {
            items(results) { word ->
                Card(modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp), elevation = 2.dp) {
                    Text(
                        text = word,
                        modifier = Modifier.padding(16.dp),
                        style = MaterialTheme.typography.body1
                    )
                }
            }
        }
    }
}
