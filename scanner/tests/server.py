import os
import random
import socket
import string
import fetch
import sys
import threading
import hashlib
import rsa
import Database
import Mailing
import dotenv
import time
import datetime
import Recognize
import base64
from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes
from cryptography.hazmat.primitives import padding

CONFIG = dotenv.dotenv_values('src/.env')


def protocol(name):
    def inner(method):
        setattr(method, 'registered', True)
        setattr(method, 'name', name)
        return method

    return inner


RSA_PUBLIC_KEY = 'rsa_public_key'
RSA_PRIVATE_KEY = 'rsa_private_key'
AUTHORIZATION_CODE = 'authorization_code'
AUTHENTICATION_TOKEN = 'authentication_token'


SEP = b"8===D<"
MESSAGE_END = b"###"
MESSAGE_HALF = b"!==!"


def extract_parameters(data: bytes) -> dict[str, bytes]:
    request_parameters = {}
    for parameter in data.split(b"~~~"):
        if SEP in parameter:
            parameter = parameter.split(SEP)
            name, value = parameter
            request_parameters[base64.b64decode(name).decode()] = base64.b64decode(value)
    return request_parameters


def create_message(sender: bytes, method: bytes, parameters: dict[bytes, bytes]):
    message = sender + b" " + method + b"~~~"
    for key, value in parameters.items():
        message += base64.b64encode(key) + SEP + base64.b64encode(value) + b"~~~"
    message += MESSAGE_END
    return message


def generate_random_string(length: int):
    return ''.join(random.choice(string.ascii_letters + string.digits) for _ in range(length))


class Server:
    CLIENT_LIMIT = 100
    PORT = 1337
    KNOWN_CLIENTS = {b"USER", b"CMRA"}
    KNOWN_REQUESTS = {}

    def __init__(self):
        self._client_count = 0
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        if sys.platform[:5] == 'linux':
            sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEPORT, 1)
            # This code is brought to you by
            # https://stackoverflow.com/questions/4465959/python-errno-98-address-already-in-use
        self._sock = sock
        self._clients = {}
        mailer = Mailing.Mailer('smtp.gmail.com', False)
        mailer.enter_credentials(CONFIG['MAIL_ADDR'], CONFIG['MAIL_PASS'])
        self._mailer = mailer

        for method in dir(self):
            f = getattr(self, method)
            if callable(f) and getattr(f, 'registered', False):
                self.KNOWN_REQUESTS[getattr(f, 'name')] = f

    def _do_handshake(self, client_id, client, data):
        pubkey, privkey = rsa.newkeys(1024)
        self._clients[client_id][RSA_PRIVATE_KEY] = privkey
        request_parameters = extract_parameters(data)
        n = int.from_bytes(request_parameters['NVALUE'], 'big')
        e = int.from_bytes(request_parameters['EVALUE'], 'big')
        client_public_key = rsa.PublicKey(n, e)
        print(request_parameters)
        try:
            rsa.verify(request_parameters['MESSAGE'], request_parameters['SIGNATURE'], client_public_key)
        except rsa.pkcs1.VerificationError:
            logger.error("Very bad")
        rand_message = os.urandom(16)
        signature = rsa.sign(rand_message, privkey, 'SHA-256')
        msg = create_message(b"SRVR", b"XOR", {
            b"NVALUE": pubkey.n.to_bytes((pubkey.n.bit_length() + 7) // 8, 'big'),
            b"EVALUE": pubkey.e.to_bytes((pubkey.n.bit_length() + 7) // 8, 'big'),
            b"MESSAGE": rand_message,
            b"SIGNATURE": signature
        })
        client.send(msg)
        self._clients[client_id][RSA_PUBLIC_KEY] = client_public_key

    @protocol(b"LOGIN")
    def _login(self, client_id, client, data):
        try:
            clients_public_key = self._clients[client_id][RSA_PUBLIC_KEY]
        except KeyError as error:
            logger.error(str(error))
            raise

        parameters = extract_parameters(data)
        db = Database.PlateGateDB()
        identifier = parameters['IDENTIFIER'].decode()
        password = parameters['PASSWORD'].decode()
        salt = db.select_salt(identifier)
        hashed_passowrd = hashlib.sha256((password + salt).encode()).hexdigest()
        db_hashed_password = db.get_hashed_password('users', identifier)
        succeed = hashed_passowrd == db_hashed_password
        if succeed:
            raw_token = identifier + ':' + password
            token = hashlib.sha256(raw_token.encode()).hexdigest()
            self._clients[client_id][AUTHENTICATION_TOKEN] = token
            code = generate_random_string(128)
            msg = create_message(b"SRVR", b"LOGIN", {
                b"SUCCESS": succeed.to_bytes(succeed.bit_length(), 'big'),
                b"AUTHORIZATION_CODE": code.encode()
            })
            self._clients[client_id][AUTHORIZATION_CODE] = code
        else:
            msg = create_message(b"SRVR", b"LOGIN", {
                b"SUCCESS": succeed.to_bytes(succeed.bit_length(), 'big')
            })
        encrypted = self._prepare_message(msg, b"LOGIN", clients_public_key)
        client.send(encrypted)

    @protocol(b"SIGNUP")
    def _signup(self, client_id, client, data):
        print("SIGNUP")
        try:
            client_public_key = self._clients[client_id][RSA_PUBLIC_KEY]
        except KeyError as error:
            logger.error(str(error))
            return
        parameters = extract_parameters(data)
        identifier = parameters['IDENTIFIER'].decode()
        fname = parameters['FNAME'].decode()
        lname = parameters['LNAME'].decode()
        password = parameters['PASSWORD'].decode()
        company_id = parameters['COMPANY_ID'].decode()
        email = parameters['EMAIL'].decode()
        db = Database.PlateGateDB()
        if db.insert_into('users',
                          id_number=identifier,
                          fname=fname,
                          lname=lname,
                          password=password,
                          company_id=company_id,
                          email=email,
                          user_state=1):
            raw_token = identifier + ':' + password
            token = hashlib.sha256(raw_token.encode()).hexdigest()
            self._clients[client_id][AUTHENTICATION_TOKEN] = token
            code = generate_random_string(128)
            self._clients[client_id][AUTHORIZATION_CODE] = code
            msg = create_message(b"SRVR", b"SIGNUP", {
                b"SUCCESS": True.to_bytes(True.bit_length(), 'big'),
                b"AUTHORIZATION_CODE": code.encode()
            })
            currtime = str(datetime.datetime.utcnow())
            self._mailer.mailto([email], "Don't reply (PlateGate)", "You signed up successfully at " + currtime)
        else:
            msg = create_message(b"SRVR", b"SIGNUP", {
                b"SUCCESS": False.to_bytes(False.bit_length(), 'big')
            })
        encrypted = self._prepare_message(msg, b"SIGNUP", client_public_key)
        print(encrypted)
        client.send(encrypted)
        manager_mail = db.get_manager_email_by_company_id(company_id)
        self._mailer.mailto([manager_mail],
                            "New User Signed Into you company",
                            f"A user with the id of {identifier} signed up with your company id.\n"
                            f"you can always remove him from the desktop app"
                            )
        print("SIGNUP END")

    @protocol(b"USERINFO")
    def _user_info(self, client_id, client, data):
        print("USERINFO")
        try:
            client_public_key = self._clients[client_id][RSA_PUBLIC_KEY]
        except KeyError as error:
            logger.error(str(error))
            return
        parameters = extract_parameters(data)
        auth_code = self._clients[client_id].pop(AUTHORIZATION_CODE)
        if auth_code != parameters['AUTHORIZATION_CODE'].decode():
            msg = create_message(b"SRVR", b"USERINFO", {
                b"SUCCESS": False.to_bytes(False.bit_length(), 'big'),
                b"REASON": b"UNAUTHORIZED"
            })
            encrypted = self._prepare_message(msg, b"UPDATE", client_public_key)
            client.send(encrypted)
            print("INCORRECT AUTH CODE")
            return
        db = Database.PlateGateDB()
        user = db.get_user_by_id(parameters['IDENTIFIER'].decode())
        company_name, company_id = db.get_company_by_user_id(parameters['IDENTIFIER'].decode())
        if int(user['user_state']) < 0:
            msg = create_message(b"SRVR", b"USERINFO", {
                b"SUCCESS": False.to_bytes(False.bit_length(), 'big'),
                b"REASON": b"USER IS DELETED"
            })
        else:
            msg = create_message(b"SRVR", b"USERINFO", {
                b"SUCCESS": True.to_bytes(True.bit_length(), 'big'),
                b"IDENTIFIER": str(user['id_number']).encode(),
                b"FNAME": user['fname'].encode(),
                b"LNAME": user['lname'].encode(),
                b"COMPANY_NAME": company_name.encode(),
                b"COMPANY_ID": str(company_id).encode(),
                b"EMAIL": user['email'].encode(),
                b"STATE": user['user_state'].to_bytes(user['user_state'].bit_length(), 'big')
            })
        encrypted = self._prepare_message(msg, b"USERINFO", client_public_key)
        client.send(encrypted)

    @protocol(b"MAILMANAGER")
    def _mail_manager(self, client_id, client, data):
        print('Mail MANAGER')
        try:
            client_public_key = self._clients[client_id][RSA_PUBLIC_KEY]
            client_token = self._clients[client_id][AUTHENTICATION_TOKEN]
        except KeyError as error:
            logger.error(str(error))
            return
        parameters = extract_parameters(data)
        token = parameters['AUTH_TOKEN'].decode()
        if token != client_token:
            msg = create_message(b"SRVR", b"MAILMANAGER", {
                b"SUCCESS": False.to_bytes(False.bit_length(), 'big'),
                b"REASON": b"CLIENT TOKEN IS FAULTY"
            })
            encrypted = self._prepare_message(msg, b"UPDATE", client_public_key)
            client.send(encrypted)
            print('Faulty Token')
            return
        message = parameters['MESSAGE'].decode()
        client_id_number = parameters['IDENTIFIER']
        db = Database.PlateGateDB()
        _, client_company_id = db.get_company_by_user_id(client_id_number.decode())
        print(client_company_id)
        client_manager = db.get_manager_by_company_id(client_company_id)
        manager_email = db.get_email(client_manager)
        self._mailer.mailto([manager_email], f'Message from {client_id_number.decode()} (PlateGate)', message)
        msg = create_message(b"SRVR", b"MAILMANAGER", {
            b"SUCCESS": True.to_bytes(True.bit_length(), 'big')
        })
        encrypted = self._prepare_message(msg, b"MAILMANAGER", client_public_key)
        client.send(encrypted)

    @protocol(b"DELETE")
    def _delete_user(self, client_id, client, data):
        print("DELETE UsER STARTED")
        try:
            client_public_key = self._clients[client_id][RSA_PUBLIC_KEY]
            client_token = self._clients[client_id][AUTHENTICATION_TOKEN]
        except KeyError as error:
            logger.error(str(error))
            return
        parameters = extract_parameters(data)
        token = parameters['AUTH_TOKEN'].decode()
        if token != client_token:
            msg = create_message(b"SRVR", b"DELETE", {
                b"SUCCESS": False.to_bytes(False.bit_length(), 'big'),
                b"REASON": b"CLIENT TOKEN IS FAULTY"
            })
            encrypted = self._prepare_message(msg, b"UPDATE", client_public_key)
            client.send(encrypted)
            print('Faulty Token')
            return

        db = Database.PlateGateDB()
        identifier = parameters['IDENTIFIER'].decode()
        user = db.get_user_by_id(identifier)
        manager_id = db.get_manager_by_company_id(user['company_id'])
        print(manager_id)

        if str(manager_id) != parameters['MANAGER_ID'].decode():
            msg = create_message(b"SRVR", b"UPDATE", {
                b"SUCCESS": False.to_bytes(False.bit_length(), 'big'),
                b"REASON": b"CLIENT IS NOT IN YOUR COMPANY"
            })
            encrypted = self._prepare_message(msg, b"DELETE", client_public_key)
            client.send(encrypted)
            print('Client not in company')
            return
        success = db.remove_user(identifier)
        if success:
            msg = create_message(b"SRVR", b"DELETE", {
                b"SUCCESS": success.to_bytes(success.bit_length(), 'big')
            })
        else:
            msg = create_message(b"SRVR", b"DELETE", {
                b"SUCCESS": success.to_bytes(success.bit_length(), 'big'),
                b"REASON": b"Couldn't delete client"
            })
        encrypted = self._prepare_message(msg, b"DELETE", client_public_key)
        client.send(encrypted)

    @protocol(b'UPDATE')
    def _update_user(self, client_id, client, data):
        print("UPDATE UsER STARTED")
        try:
            client_public_key = self._clients[client_id][RSA_PUBLIC_KEY]
            client_token = self._clients[client_id][AUTHENTICATION_TOKEN]
        except KeyError as error:
            logger.error(str(error))
            return
        parameters = extract_parameters(data)
        print(parameters)
        token = parameters['AUTH_TOKEN'].decode()
        if token != client_token:
            msg = create_message(b"SRVR", b"UPDATE", {
                b"SUCCESS": False.to_bytes(False.bit_length(), 'big'),
                b"REASON": b"CLIENT TOKEN IS FAULTY"
            })
            encrypted = self._prepare_message(msg, b"UPDATE", client_public_key)
            client.send(encrypted)
            print('Faulty Token')
            return

        db = Database.PlateGateDB()
        identifier = parameters['IDENTIFIER'].decode()
        user = db.get_user_by_id(identifier)
        manager_id = db.get_manager_by_company_id(user['company_id'])
        print(manager_id)

        if str(manager_id) != parameters['MANAGER_ID'].decode():
            msg = create_message(b"SRVR", b"UPDATE", {
                b"SUCCESS": False.to_bytes(False.bit_length(), 'big'),
                b"REASON": b"CLIENT IS NOT IN YOUR COMPANY"
            })
            encrypted = self._prepare_message(msg, b"UPDATE", client_public_key)
            client.send(encrypted)
            print('Client not in company')
            return
        success = db.update('users',
                            id_number=identifier,
                            fname=parameters.get('FNAME', user['fname']),
                            lname=parameters.get('LNAME', user['lname']),
                            email=parameters.get('EMAIL', user['email']),
                            )
        if success:
            msg = create_message(b"SRVR", b"UPDATE", {
                b"SUCCESS": success.to_bytes(success.bit_length(), 'big')
            })
        else:
            msg = create_message(b"SRVR", b"UPDATE", {
                b"SUCCESS": success.to_bytes(success.bit_length(), 'big'),
                b"REASON": b"Couldn't commit changes"
            })
        encrypted = self._prepare_message(msg, b"UPDATE", client_public_key)
        client.send(encrypted)

    @protocol(b"RECOGNIZE")
    def _recognize(self, client_id, client, data):
        try:
            client_public_key = self._clients[client_id][RSA_PUBLIC_KEY]
        except KeyError as error:
            logger.error(str(error))
            return

        filename = f'ServerImages/{client_id}.jpg'
        print(filename)
        parameters = extract_parameters(data)
        image_bytes = parameters['IMAGE']
        with open(filename, 'wb') as f:
            f.write(image_bytes)
        license_plate = Recognize.recognize_from_image(filename)
        print(license_plate)
        if not license_plate:
            message = create_message(b"SRVR", b"RECOGNIZE", {
                b"SUCCESS": False.to_bytes(False.bit_length(), 'big'),
                b"REASON": b"bad image, could not recognize"
            })
            encrypted = self._prepare_message(message, b"RECOGNIZE", client_public_key)
            client.send(encrypted)
            logger.info("couldnt recognize image")
            return
        db = Database.PlateGateDB()
        vehicle_db = db.get_vehicle_by_plate_number(license_plate)
        if not vehicle_db:
            message = create_message(b"SRVR", b"RECOGNIZE", {
                b"SUCCESS": False.to_bytes(False.bit_length(), 'big'),
                b"REASON": b"vehicle not in database"
            })
            encrypted = self._prepare_message(message, b"RECOGNIZE", client_public_key)
            client.send(encrypted)
            logger.info("couldnt recognize image")
            return
        owner_id = vehicle_db['owner_id']
        name, company_id = db.get_company_by_user_id(owner_id)
        if int.from_bytes(parameters['COMPANY_ID'], 'big') != company_id:
            message = create_message(b"SRVR", b"RECOGNIZE", {
                b"SUCCESS": False.to_bytes(False.bit_length(), 'big'),
                b"REASON": b"car not in company"
            })
            encrypted = self._prepare_message(message, b"RECOGNIZE", client_public_key)
            client.send(encrypted)
            logger.info("couldnt recognize image")
            return
        vehicle_gov = fetch.GovApiFetcher.get_vehicle_by_plate_number(license_plate)
        validation_list = [
            vehicle_gov.plate_number == vehicle_db['plate_number'],
            vehicle_gov.shnat_yitsur == vehicle_db['shnat_yitsur'],
            vehicle_gov.sug_delek.value == vehicle_db['sug_delek'],
            vehicle_gov.sug_rechev.value == vehicle_db['sug_rechev'],
            vehicle_db['vehicle_state'] >= 0,
            vehicle_gov.active(),
            not vehicle_gov.totaled
        ]  # Validate vehicle with data.gov.il
        is_valid = all(validation_list)
        if not is_valid:
            print('vehicle not valid')
            message = create_message(b"SRVR", b"RECOGNIZE", {
                b"SUCCESS": False.to_bytes(False.bit_length(), 'big'),
                b"REASON": f'{validation_list}'.encode()
            })
            encrypted = self._prepare_message(message, b"RECOGNIZE", client_public_key)
            client.send(encrypted)
            logger.info('vehicle not valid')
            return
        recognized = f'./ServerImages/{client_id}_RECOGNIZED.png'
        with open(recognized, 'rb') as f:
            image_bytes = f.read()
        os.remove(recognized)
        message = create_message(b"SRVR", b"RECOGNIZE", {
            b"SUCCESS": True.to_bytes(True.bit_length(), 'big'),
            b"RECOGNIZED_IMAGE": image_bytes
        })
        encrypted = self._prepare_message(message, b"RECOGNIZE", client_public_key)
        client.send(encrypted)
        if sys.platform.startswith('linux'):
            time_now = time.clock_gettime(time.CLOCK_REALTIME)
        else: time_now = time.time()
        time_readalbe = datetime.datetime.now()
        print(time_readalbe)
        inserted = db.insert_into('entries',
                                  time_now=time_now,
                                  time_readable=str(time_readalbe),
                                  person_id=vehicle_db['owner_id'],
                                  car_id=vehicle_db['plate_number'],
                                  company_id=company_id)
        if inserted:
            print('entry inserted')
        else:
            print('entry not inserted')

    @protocol(b"ADDPLATE")
    def _add_plate(self, client_id, client, data):
        try:
            client_public_key = self._clients[client_id][RSA_PUBLIC_KEY]
        except KeyError:
            return
        parameters = extract_parameters(data)
        manager_id = parameters['MANAGER_ID'].decode()
        user_id = parameters['USER_ID'].decode()
        plate_number = parameters['PLATE_NUMBER'].decode()
        db = Database.PlateGateDB()
        user_company = db.get_company_by_user_id(user_id)
        manager_company = db.get_company_by_user_id(manager_id)
        if manager_company != user_company:
            message = create_message(b"SRVR", b"ADDPLATE", {
                b"SUCCESS": False.to_bytes(False.bit_length(), 'big'),
                b"REASON": b"CLIENT NOT IN COMPANY"
            })
            encrypted = self._prepare_message(message, b"ADDPLATE", client_public_key)
            client.send(encrypted)
            return
        manager_state = int(db.get_user_by_id(manager_id)['user_state'])
        if manager_state < 2:
            message = create_message(b"SRVR", b"ADDPLATE", {
                b"SUCCESS": False.to_bytes(False.bit_length(), 'big'),
                b"REASON": b"YOURE NOT A MANAGER"
            })
            encrypted = self._prepare_message(message, b"ADDPLATE", client_public_key)
            client.send(encrypted)
            return
        vehicle = fetch.GovApiFetcher.get_vehicle_by_plate_number(plate_number)
        if not vehicle:
            message = create_message(b"SRVR", b"ADDPLATE", {
                b"SUCCESS": False.to_bytes(False.bit_length(), 'big'),
                b"REASON": b"VEHICLE DOTN EXIST"
            })
            encrypted = self._prepare_message(message, b"ADDPLATE", client_public_key)
            client.send(encrypted)
            return
        if vehicle.totaled or not vehicle.active():
            message = create_message(b"SRVR", b"ADDPLATE", {
                b"SUCCESS": False.to_bytes(False.bit_length(), 'big'),
                b"REASON": b"VEHICLE IS NOT ACTIVE OR TOTAL LOSS"
            })
            encrypted = self._prepare_message(message, b"ADDPLATE", client_public_key)
            client.send(encrypted)
            return
        user_state = db.get_user_by_id(user_id)['user_state']
        updated = db.update('vehicles', plate_number=plate_number, vehicle_state=1)
        if not updated:
            message = create_message(b"SRVR", b"ADDPLATE", {
                b"SUCCESS": False.to_bytes(False.bit_length(), 'big'),
                b"REASON": b"Couldnt enter vehicle to db"
            })
            encrypted = self._prepare_message(message, b"ADDPLATE", client_public_key)
            client.send(encrypted)
            return
        inserted = vehicle.add_to_database(user_id)
        if not inserted and not updated:
            message = create_message(b"SRVR", b"ADDPLATE", {
                b"SUCCESS": False.to_bytes(False.bit_length(), 'big'),
                b"REASON": b"Couldnt enter vehicle to db"
            })
            encrypted = self._prepare_message(message, b"ADDPLATE", client_public_key)
            client.send(encrypted)
            return
        if user_state == 0:
            db.update('vehicles', plate_number=plate_number, vehicle_state=0)
        print('WE MADE IT')
        message = create_message(b"SRVR", b"ADDPLATE", {
            b"SUCCESS": True.to_bytes(True.bit_length(), 'big')
        })
        encrypted = self._prepare_message(message, b"ADDPLATE", client_public_key)
        client.send(encrypted)
        return

    @protocol(b"REMOVEPLATE")
    def _remove_plate(self, client_id, client, data):
        try:
            client_public_key = self._clients[client_id][RSA_PUBLIC_KEY]
        except KeyError:
            return
        parameters = extract_parameters(data)
        manager_id = parameters['MANAGER_ID'].decode()
        user_id = parameters['USER_ID'].decode()
        plate_number = parameters['PLATE_NUMBER'].decode()
        db = Database.PlateGateDB()
        user_company = db.get_company_by_user_id(user_id)
        manager_company = db.get_company_by_user_id(manager_id)
        if manager_company != user_company:
            message = create_message(b"SRVR", b"REMOVEPLATE", {
                b"SUCCESS": False.to_bytes(False.bit_length(), 'big'),
                b"REASON": b"CLIENT NOT IN COMPANY"
            })
            encrypted = self._prepare_message(message, b"REMOVEPLATE", client_public_key)
            client.send(encrypted)
            return
        manager_state = int(db.get_user_by_id(manager_id)['user_state'])
        if manager_state < 2:
            message = create_message(b"SRVR", b"REMOVEPLATE", {
                b"SUCCESS": False.to_bytes(False.bit_length(), 'big'),
                b"REASON": b"YOURE NOT A MANAGER"
            })
            encrypted = self._prepare_message(message, b"REMOVEPLATE", client_public_key)
            client.send(encrypted)
            return
        vehicle_in_database = db.get_vehicle_by_plate_number(plate_number)
        if vehicle_in_database['vehicle_state'] < 0:
            message = create_message(b"SRVR", b"REMOVEPLATE", {
                b"SUCCESS": False.to_bytes(False.bit_length(), 'big'),
                b"REASON": b"Vehicle is already deleted"
            })
            encrypted = self._prepare_message(message, b"REMOVEPLATE", client_public_key)
            client.send(encrypted)
            return
        deleted = db.remove_plate(plate_number)
        if not deleted:
            message = create_message(b"SRVR", b"REMOVEPLATE", {
                b"SUCCESS": False.to_bytes(False.bit_length(), 'big'),
                b"REASON": b"Couldnt delete vehicle in db"
            })
            encrypted = self._prepare_message(message, b"REMOVEPLATE", client_public_key)
            client.send(encrypted)
            return
        print('WE MADE IT')
        message = create_message(b"SRVR", b"REMOVEPLATE", {
            b"SUCCESS": True.to_bytes(True.bit_length(), 'big')
        })
        encrypted = self._prepare_message(message, b"REMOVEPLATE", client_public_key)
        client.send(encrypted)

    @protocol(b'ADDCOMPANY')
    def _add_company(self, client_id, client, data):
        try:
            client_public_key = self._clients[client_id][RSA_PUBLIC_KEY]
        except KeyError:
            return
        parameters = extract_parameters(data)
        db = Database.PlateGateDB()

        inserted_user = db.insert_into('users',
                                       id_number=parameters['IDENTIFIER'].decode(),
                                       fname=parameters['FNAME'].decode(),
                                       lname=parameters['LNAME'].decode(),
                                       password=parameters['PASSWORD'].decode(),
                                       email=parameters['EMAIL'].decode(),
                                       user_state=2)

        if not inserted_user:
            msg = create_message(b"SRVR", b"ADDCOMPANY", {
                b"SUCCESS": False.to_bytes(False.bit_length(), 'big'),
                b"REASON": b"Couldn't insert client"
            })
            encrypted = self._prepare_message(msg, b"ADDCOMPANY", client_public_key)
            client.send(encrypted)
            return
        inserted_company = db.insert_into('companies',
                                          company_name=parameters['COMPANY_NAME'].decode(),
                                          manager_id=parameters['IDENTIFIER'].decode())
        if not inserted_company:
            msg = create_message(b"SRVR", b"ADDCOMPANY", {
                b"SUCCESS": False.to_bytes(False.bit_length(), 'big'),
                b"REASON": b"Couldn't insert comppany"
            })
            encrypted = self._prepare_message(msg, b"ADDCOMPANY", client_public_key)
            client.send(encrypted)
            return
        company_id = db.get_company_by_manager_id(parameters['IDENTIFIER'].decode())
        updated = db.update('users',
                            id_number=parameters['IDENTIFIER'].decode(),
                            company_id=company_id)
        if not updated:
            msg = create_message(b"SRVR", b"ADDCOMPANY", {
                b"SUCCESS": False.to_bytes(False.bit_length(), 'big'),
                b"REASON": b"Couldn't update user"
            })
            encrypted = self._prepare_message(msg, b"ADDCOMPANY", client_public_key)
            client.send(encrypted)
            return

        msg = create_message(b"SRVR", b"ADDCOMPANY", {
            b"SUCCESS": True.to_bytes(True.bit_length(), 'big'),
            b"COMPANY_ID": company_id.to_bytes(company_id.bit_length(), 'big')
        })
        encrypted = self._prepare_message(msg, b"ADDCOMPANY", client_public_key)
        client.send(encrypted)
        return

    @protocol(b'GETENTRIES')
    def _get_entries(self, client_id, client, data):
        try:
            client_public_key = self._clients[client_id][RSA_PUBLIC_KEY]
        except KeyError:
            return
        parameters = extract_parameters(data)
        db = Database.PlateGateDB()
        requesting_user_id = parameters['MANAGER_ID'].decode()
        requesting_user = db.get_user_by_id(requesting_user_id)
        if requesting_user['user_state'] < 2:
            msg = create_message(b'SRVR', b'GETENTRIES', {
                b"SUCCESS": False.to_bytes(False.bit_length(), 'big'),
                b"REASON": b"Unauthorized to do such actions"
            })
            encrypted = self._prepare_message(msg, b"GETENTRIES", client_public_key)
            client.send(encrypted)
            return
        company_name, company_id = db.get_company_by_user_id(requesting_user_id)
        entries = db.get_entries_by_company_id(company_id)
        def get_full_name_of_user(user_id):
            user = db.get_user_by_id(user_id)
            return f"{user['fname']} {user['lname']}"

        for entry in entries:
            entry.pop('company_id')
            entry['company_name'] = company_name
            entry['full_name'] = get_full_name_of_user(entry['person_id'])

        port = 1357
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        if sys.platform[:5] == 'linux':
            sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEPORT, 1)
        sock.bind(('0.0.0.0', port))
        sock.listen()
        message = create_message(b"SRVR", b"GETENTRIES", {
            b"SUCCESS": True.to_bytes(True.bit_length(), 'big'),
            b"STATE": b"SENDING_ENTRIES"
        })
        encrypted = self._prepare_message(message, b"GETENTRIES", client_public_key)
        client.send(encrypted)
        entries_client, client_addr = sock.accept()
        new_client_id = hashlib.sha256(str(client_addr[0]).encode()).hexdigest()
        while new_client_id != client_id:
            entries_client.close()
            entries_client, client_addr = sock.accept()
            new_client_id = hashlib.sha256(str(client_addr[0]).encode()).hexdigest()
        for entry in entries:
            message = create_message(b"SRVR", b"GETENTRIES", {
                b"ENTRY_ID": str(entry['entry_id']).encode(),
                b"TIME": entry['time_readable'].encode(),
                b"PERSON_ID": str(entry['person_id']).encode(),
                b"PERSON_NAME": entry['full_name'].encode(),
                b"PLATE_NUMBER": str(entry['car_id']).encode(),
                b"COMPANY_NAME": entry['company_name'].encode()
            })
            encrypted = self._prepare_message(message, b"GETENTRIES", client_public_key)
            entries_client.send(encrypted)
            time.sleep(0.01)

        message = create_message(b"SRVR", b"GETENTRIES", {
            b"SUCCESS": True.to_bytes(True.bit_length(), 'big'),
            b"STATE": b"FINISHED"
        })
        encrypted = self._prepare_message(message, b"GETENTRIES", client_public_key)
        entries_client.send(encrypted)
        time.sleep(0.1)

    @staticmethod
    def _prepare_message(message: bytes, method: bytes, client_public_key) -> bytes:
        padder = padding.PKCS7(128).padder()
        padded_msg = padder.update(message) + padder.finalize()

        aes_key = os.urandom(16)
        iv = os.urandom(16)
        aes_info = create_message(b"SRVR", method, {
            b"AES_KEY": aes_key,
            b"IVECTOR": iv
        })
        encrypted_info = rsa.encrypt(aes_info, client_public_key)
        aes = Cipher(algorithms.AES(aes_key), mode=modes.CBC(iv))
        encryptor = aes.encryptor()
        encrypted_message = encryptor.update(padded_msg) + encryptor.finalize()
        return encrypted_info + MESSAGE_HALF + encrypted_message + MESSAGE_END

    def _handle_client(self, client, addr):
        client_id = hashlib.sha256(str(addr[0]).encode()).hexdigest()
        if client_id not in self._clients.keys():
            self._clients[client_id] = {}
        data = client.recv(1024)
        while not data[-len(MESSAGE_END):] == MESSAGE_END:
            data += client.recv(1024)

        data = data[:-len(MESSAGE_END)]

        if MESSAGE_HALF in data:
            info, content = data.split(MESSAGE_HALF)
        else:
            self._do_handshake(client_id, client, data)
            return

        decrypted_info = rsa.decrypt(info, self._clients[client_id][RSA_PRIVATE_KEY])
        parameters = extract_parameters(decrypted_info)

        aes_key = parameters['AES_KEY']
        iv = parameters['IVECTOR']

        aes = Cipher(algorithms.AES(aes_key), mode=modes.CBC(iv))
        decryptor = aes.decryptor()
        unpadder = padding.PKCS7(128).unpadder()

        decrypted = decryptor.update(content) + decryptor.finalize()
        decrypted = unpadder.update(decrypted) + unpadder.finalize()

        if not decrypted_info[:4] in self.KNOWN_CLIENTS:
            client.close()
            self._client_count -= 1
            return

        function = decrypted_info.split(b"~~~")[0].split(b" ")[1]
        print(function)
        if function not in self.KNOWN_REQUESTS:
            print('function not exists')
            print('closing client')
            client.close()
            self._client_count -= 1
            return

        self.KNOWN_REQUESTS[function](client_id, client, decrypted)
        client.close()
        self._client_count -= 1

    def mainloop(self):
        self._sock.bind(('0.0.0.0', self.PORT))
        self._sock.listen()
        threads = []
        while self._client_count <= self.CLIENT_LIMIT:
            print('Waiting')
            client, addr = self._sock.accept()
            thread = threading.Thread(target=self._handle_client, args=(client, addr,))
            threads.append(thread)
            thread.start()
            self._client_count += 1

        for thread in threads:
            thread.join()

        self._sock.close()


server = Server()
server.mainloop()