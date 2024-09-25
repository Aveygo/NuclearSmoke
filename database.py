
from peewee import *

db = SqliteDatabase('main.db')

class Fire(Model):
    lat = FloatField()
    long = FloatField()

    title = CharField()
    published = IntegerField()
    category = CharField()
    updated = IntegerField()
    created = IntegerField()
    level = CharField()
    size_ha = FloatField() # Fire size in hectares

    temp = FloatField() # Celsius
    under_control = BooleanField()

    wind_speed = FloatField() # km/h
    wind_direction = FloatField() # degrees (from north=0)
    wind_shear = FloatField() # average change in horizontal wind speed (kmph) per height (m) 
    
    class Meta:
        database = db

class Contour(Model):
    data = CharField() # Json 2d array of points [[0, 1], [2, 3] ... ]
    thresh = FloatField()

    owner = ForeignKeyField(Fire, backref='contours')

    class Meta:
        database = db

db.connect()
db.create_tables([Fire, Contour])
