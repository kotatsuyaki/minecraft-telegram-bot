package tk.akitaki.tgbot;

import cpw.mods.fml.common.Mod;
import cpw.mods.fml.common.Mod.EventHandler;
import cpw.mods.fml.common.event.FMLInitializationEvent;
import cpw.mods.fml.common.event.FMLServerStoppingEvent;

import net.minecraftforge.common.MinecraftForge;
import cpw.mods.fml.common.eventhandler.SubscribeEvent;

import cpw.mods.fml.common.FMLCommonHandler;
import net.minecraftforge.event.ServerChatEvent;
import cpw.mods.fml.common.gameevent.PlayerEvent.PlayerLoggedInEvent;
import cpw.mods.fml.common.gameevent.PlayerEvent.PlayerLoggedOutEvent;
import net.minecraft.entity.player.EntityPlayer;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;

import java.util.HashMap;
import java.util.List;
import java.util.ArrayList;
import java.util.Scanner;
import java.lang.Thread;
import java.lang.Process;
import java.lang.ProcessBuilder;
import java.nio.charset.Charset ;

import java.io.BufferedReader;
import java.io.InputStreamReader;

import net.minecraft.server.MinecraftServer;
import net.minecraft.util.ChatComponentText;


@Mod(modid = TgBotMod.MODID, version = TgBotMod.VERSION, acceptableRemoteVersions = "*")
public class TgBotMod {
    public static final String MODID = "tgbotmod";
    public static final String VERSION = "1.0";

    private EventCallbackHandler eventCallbackHandler;
    private Process botProcess;
    private BotInputThread botInputThread;

    @EventHandler
    public void init(FMLInitializationEvent event) throws java.io.IOException {
        startBot();

        eventCallbackHandler = new EventCallbackHandler(this);

        botInputThread = new BotInputThread(this);
        botInputThread.start();

        MinecraftForge.EVENT_BUS.register(eventCallbackHandler);
        // this is needed for PlayerLoggedIn(Out)Event to fire
        FMLCommonHandler.instance().bus().register(eventCallbackHandler);
    }

    @EventHandler
    public void stop(FMLServerStoppingEvent event) throws java.io.IOException {
        botProcess.destroy();
    }

    private void startBot() throws java.io.IOException {
        System.out.println("Starting bot process");

        List<String> botProcessCmd = new ArrayList<String>();
        botProcessCmd.add("python3");
        botProcessCmd.add("./bot.py");

        botProcess = new ProcessBuilder(botProcessCmd).start();
    }

    public void onPlayerChat(ServerChatEvent event) {
        HashMap<String, Object> map = new HashMap<>();
        map.put("event", "chat_msg");
        map.put("name", event.username);
        map.put("msg", event.message);

        printToBotProcess(map);
    }

    public void onPlayerJoin(EntityPlayer player) {
        HashMap<String, Object> map = new HashMap<>();
        map.put("event", "player_join");
        map.put("name", player.getGameProfile().getName());

        printToBotProcess(map);
    }

    public void onPlayerLeave(EntityPlayer player) {
        HashMap<String, Object> map = new HashMap<>();
        map.put("event", "player_leave");
        map.put("name", player.getGameProfile().getName());

        printToBotProcess(map);
    }

    private void printToBotProcess(HashMap<String, Object> map) {
        Gson gson = new Gson();
        String jsonString = gson.toJson(map) + "\n";

        try {
            botProcess.getOutputStream().write(jsonString.getBytes(Charset.forName("UTF-8")));
            botProcess.getOutputStream().flush();
        } catch (java.io.IOException e) {
            System.out.println("Failed to print to bot process");
        }
    }

    public class BotInputThread extends Thread {
        private TgBotMod mod;
        public BotInputThread(TgBotMod mod) {
            this.mod = mod;
        }

        public void run() {
            System.out.println("Bot input thread running");
            BufferedReader reader = new BufferedReader(new InputStreamReader(mod.botProcess.getInputStream()));

            try {
                String line = reader.readLine();
                while (line != null) {
                    System.out.println("Received line from bot process: " + line);

                    Gson gson = new Gson();
                    HashMap<String, Object> event = gson.fromJson(line, HashMap.class);

                    switch ((String)event.get("event")) {
                    case "chat_msg":
                        handleChatMsg(event);
                        break;
                    }

                    line = reader.readLine();
                }
            } catch (java.io.IOException e) {
                System.out.println("Failed to get input from bot process");
            }
        }

        void handleChatMsg(HashMap<String, Object> event) {
            String name = (String)event.get("name");
            Scanner scanner = new Scanner((String)event.get("msg"));
            while (scanner.hasNextLine()) {
                String line = scanner.nextLine();
                String formatted = String.format("[@%s] %s", name, line);
                MinecraftServer
                    .getServer()
                    .getConfigurationManager()
                    .sendChatMsg(new ChatComponentText(formatted));
            }
        }
    }

    public class EventCallbackHandler {
        private TgBotMod mod;
        public EventCallbackHandler(TgBotMod mod) {
            this.mod = mod;
        }

        @SubscribeEvent
        public void onPlayerJoin(PlayerLoggedInEvent event) {
            mod.onPlayerJoin((EntityPlayer)event.player);
        }

        @SubscribeEvent
        public void onPlayerLeave(PlayerLoggedOutEvent event) {
            mod.onPlayerLeave((EntityPlayer)event.player);
        }

        @SubscribeEvent
        public void onPlayerChat(ServerChatEvent event) {
            mod.onPlayerChat(event);
        }
    }
}
