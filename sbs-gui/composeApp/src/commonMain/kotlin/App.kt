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
    val present: String,
    val repeats: Int? = null
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
    // UI State
    var serverUrl by remember { mutableStateOf("http://localhost:8080") }
    var letters by remember { mutableStateOf("") }
    var present by remember { mutableStateOf("") }
    var repeats by remember { mutableStateOf("") } // Optional input
    
    var results by remember { mutableStateOf<List<String>>(emptyList()) }
    var errorMessage by remember { mutableStateOf<String?>(null) }
    var isLoading by remember { mutableStateOf(false) }

    val scope = rememberCoroutineScope()
    
    // Create HTTP Client
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
        
        // --- Server Configuration ---
        OutlinedTextField(
            value = serverUrl,
            onValueChange = { serverUrl = it },
            label = { Text("Server URL") },
            placeholder = { Text("http://34.x.x.x:80") },
            modifier = Modifier.fillMaxWidth(),
            colors = TextFieldDefaults.outlinedTextFieldColors(
                focusedBorderColor = MaterialTheme.colors.primary.copy(alpha = 0.5f)
            )
        )
        
        Divider(modifier = Modifier.padding(vertical = 5.dp))

        // --- Puzzle Configuration ---
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

        OutlinedTextField(
            value = repeats,
            onValueChange = { 
                // Only allow numeric input
                if (it.all { char -> char.isDigit() }) {
                    repeats = it 
                }
            },
            label = { Text("Max Repeats (Optional)") },
            modifier = Modifier.fillMaxWidth()
        )
        
        Button(
            onClick = {
                scope.launch {
                    isLoading = true
                    errorMessage = null
                    results = emptyList() // Clear previous results
                    try {
                        // Ensure endpoint has /solve
                        val baseUrl = serverUrl.trimEnd('/')
                        val endpoint = if (baseUrl.endsWith("/solve")) baseUrl else "$baseUrl/solve"
                        
                        val repeatsInt = if (repeats.isNotEmpty()) repeats.toInt() else null

                        val response = client.post(endpoint) {
                            header("Content-Type", "application/json")
                            setBody(SolveRequest(letters, present, repeatsInt))
                        }
                        
                        if (response.status.value in 200..299) {
                            results = response.body()
                        } else {
                            errorMessage = "Error ${response.status.value}: ${response.bodyAsText()}"
                        }
                    } catch (e: Exception) {
                        errorMessage = "Connection Failed: ${e.message}"
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
        
        // --- Output ---
        if (errorMessage != null) {
            Text(errorMessage!!, color = Color.Red)
        }
        
        if (results.isNotEmpty()) {
            Text("Found ${results.size} words", style = MaterialTheme.typography.h6)
        }
        
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(bottom = 16.dp)
        ) {
            items(results) { word ->
                Card(
                    modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp), 
                    elevation = 2.dp,
                    backgroundColor = Color(0xFFF5F5F5)
                ) {
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
