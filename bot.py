from os import environ
import sys
import json

from telegram import Update
from telegram.bot import Bot
from telegram.ext import Updater, CommandHandler, CallbackContext, MessageHandler
from telegram.ext.dispatcher import Dispatcher
from telegram.ext.filters import Filters

sys.stderr = open('err.txt', 'w')

BOT_TOKEN = environ['BOT_TOKEN']
CHATID = environ['CHATID']

def id(update: Update, context: CallbackContext):
    if update.effective_chat is None:
        return
    context.bot.send_message(
        chat_id=update.effective_chat.id,
        text='Chat id is {}'.format(update.effective_chat.id))

def ping(update: Update, context: CallbackContext):
    if update.effective_chat is None:
        return
    if update.message is None:
        return

    context.bot.send_message(
        chat_id=update.effective_chat.id,
        reply_to_message_id=update.message.message_id,
        text='有ら server 有開ら')

def chat_callback(update: Update, context: CallbackContext):
    if update.message is None:
        return
    if update.message.text is None:
        return
    if update.message.from_user is None:
        return
    if update.message.from_user.username is None:
        return

    json_str = json.dumps({
        'event': 'chat_msg',
        'name': update.message.from_user.username,
        'msg': update.message.text,
    })
    print(json_str, flush=True)

def start():
    logfile = open('out.txt', 'w')
    logfile.write('Bot started\n')
    logfile.flush()

    updater = Updater(token=BOT_TOKEN, use_context=True)
    dispatcher: Dispatcher = updater.dispatcher

    bot: Bot = updater.bot
    bot.send_message(chat_id=CHATID, text='`[Server 開ら]`', parse_mode='MarkdownV2')

    dispatcher.add_handler(CommandHandler('id', id))
    dispatcher.add_handler(CommandHandler('ping', ping))
    # dispatcher.add_handler(MessageHandler(Filters.all, chat_callback))
    dispatcher.add_handler(MessageHandler(Filters.chat(int(CHATID)), chat_callback))
    updater.start_polling()

    for line in sys.stdin:
        event = json.loads(line.strip())
        logfile.write('Received event: {}\n'.format(event))
        logfile.flush()

        if event['event'] == 'player_join':
            bot.send_message(
                chat_id=CHATID,
                text='`[{} 上線ら]`'.format(event['name']),
                parse_mode='MarkdownV2'
            )
        elif event['event'] == 'player_leave':
            bot.send_message(
                chat_id=CHATID,
                text='`[{} 跑路ら]`'.format(event['name']),
                parse_mode='MarkdownV2'
            )
        elif event['event'] == 'chat_msg':
            bot.send_message(
                chat_id=CHATID,
                text='`[{}] {}`'.format(event['name'], event['msg']),
                parse_mode='MarkdownV2'
            )
        elif event['event'] == 'player_death':
            bot.send_message(
                chat_id=CHATID,
                text='`[{} dieら: {}]`'.format(event['name'], event['msg']),
                parse_mode='MarkdownV2'
            )
        else:
            logfile.write('Unknown event of kind "{}"}\n'.format(event['event']))
            logfile.flush()

    logfile.close()

if __name__ == '__main__':
    start()
