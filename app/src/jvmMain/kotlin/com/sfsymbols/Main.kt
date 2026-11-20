package com.sfsymbols

import androidx.compose.ui.graphics.vector.rememberVectorPainter
import androidx.compose.ui.Alignment
import androidx.compose.ui.unit.DpSize
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.WindowState
import androidx.compose.ui.window.WindowPosition
import com.sfsymbols.mcp.McpServer
import com.sfsymbols.ui.AppLogo
import com.sfsymbols.ui.SfSymbolsCatalogApp
import dev.nucleusframework.application.nucleusApplication
import dev.nucleusframework.macoscompose.window.MacosDecoratedWindow
import java.io.IOException
import java.nio.file.Files
import java.nio.file.StandardCopyOption
import kotlin.system.exitProcess

fun main(args: Array<String>) {
    if ("--mcp" in args) {
        runMcpServer()
        return
    }
    launchCatalog()
}

private fun runMcpServer() {
    val windowsHost = System.getProperty("os.name").startsWith("Windows", ignoreCase = true)
    val extension = if (windowsHost) ".exe" else ""
    val rustServer = if (System.getProperty("compose.application.resources.dir") != null) {
        McpServer::class.java.getResourceAsStream("/native/sfstudio-mcp$extension")
    } else {
        null
    }

    if (rustServer == null) {
        McpServer().runBlockingUntilExit()
        return
    }

    val exitCode = try {
        val executable = Files.createTempFile("sfstudio-mcp-", if (windowsHost) ".exe" else "")
        try {
            rustServer.use { Files.copy(it, executable, StandardCopyOption.REPLACE_EXISTING) }
            if (!windowsHost && !executable.toFile().setExecutable(true, true)) {
                throw IOException("Could not mark the bundled Rust MCP executable as executable")
            }
            ProcessBuilder(executable.toString()).inheritIO().start().waitFor()
        } finally {
            Files.deleteIfExists(executable)
        }
    } catch (error: IOException) {
        System.err.println("Rust MCP could not start; falling back to Kotlin MCP: ${error.message}")
        McpServer().runBlockingUntilExit()
        return
    }
    if (exitCode != 0) exitProcess(exitCode)
}

private fun launchCatalog() = nucleusApplication {
    MacosDecoratedWindow(
        onCloseRequest = { exitApplication() },
        title = "SFStudio",
        icon = rememberVectorPainter(AppLogo),
        state = WindowState(
            position = WindowPosition(Alignment.Center),
            size = DpSize(1280.dp, 820.dp),
        ),
        minimumSize = DpSize(1000.dp, 680.dp),
    ) {
        SfSymbolsCatalogApp()
    }
}
